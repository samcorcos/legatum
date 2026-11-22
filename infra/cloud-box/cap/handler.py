"""Stop the Legatum build box when it has run past the day's cap, or is up but doing nothing.

The net under the box's own idle watcher: it runs inside AWS, so a broken box, a hung job or a Mac that
is off cannot keep it running. The day's running minutes live in tags on the instance (cap-day,
cap-used-min, cap-last), so no database is needed.
"""
import datetime
import os

import boto3

ec2 = boto3.client("ec2")
cw = boto3.client("cloudwatch")
CAP = float(os.environ["DAILY_CAP_MINUTES"])
INTERVAL = float(os.environ["INTERVAL_MINUTES"])
IDLE_AFTER = datetime.timedelta(minutes=40)
IDLE_WINDOW = datetime.timedelta(minutes=30)
IDLE_CPU_PERCENT = 3.0


def tags_of(instance):
    return {t["Key"]: t["Value"] for t in instance.get("Tags", [])}


def cpu_idle(instance_id, now):
    points = cw.get_metric_statistics(
        Namespace="AWS/EC2",
        MetricName="CPUUtilization",
        Dimensions=[{"Name": "InstanceId", "Value": instance_id}],
        StartTime=now - IDLE_WINDOW,
        EndTime=now,
        Period=300,
        Statistics=["Maximum"],
    )["Datapoints"]
    # Basic monitoring has one point per 5 minutes: wait for at least five before judging.
    return len(points) >= 5 and max(p["Maximum"] for p in points) < IDLE_CPU_PERCENT


def handler(event, context):
    now = datetime.datetime.now(datetime.timezone.utc)
    today = now.strftime("%Y-%m-%d")
    stopped = []
    pages = ec2.get_paginator("describe_instances").paginate(
        Filters=[
            {"Name": "tag:project", "Values": ["legatum-box"]},
            {"Name": "tag:role", "Values": ["build-box"]},
            {"Name": "instance-state-name", "Values": ["running"]},
        ]
    )
    for page in pages:
        for reservation in page["Reservations"]:
            for instance in reservation["Instances"]:
                iid = instance["InstanceId"]
                tags = tags_of(instance)
                used = float(tags.get("cap-used-min", 0)) if tags.get("cap-day") == today else 0.0
                try:
                    last = datetime.datetime.fromtimestamp(float(tags["cap-last"]), datetime.timezone.utc)
                except (KeyError, ValueError):
                    last = instance["LaunchTime"]
                since = max(last, instance["LaunchTime"])
                # Count only time the box was running; a gap in invocations counts at most 3 intervals.
                used += min((now - since).total_seconds() / 60, INTERVAL * 3)
                ec2.create_tags(
                    Resources=[iid],
                    Tags=[
                        {"Key": "cap-day", "Value": today},
                        {"Key": "cap-used-min", "Value": f"{used:.1f}"},
                        {"Key": "cap-last", "Value": str(int(now.timestamp()))},
                    ],
                )
                reason = None
                if used >= CAP:
                    reason = f"daily cap reached ({used:.0f} of {CAP:.0f} minutes today)"
                elif now - instance["LaunchTime"] > IDLE_AFTER and cpu_idle(iid, now):
                    reason = "up with CPU under 3% for 30 minutes"
                if reason:
                    print(f"stopping {iid}: {reason}")
                    ec2.stop_instances(InstanceIds=[iid])
                    stopped.append(iid)
    return {"stopped": stopped}
