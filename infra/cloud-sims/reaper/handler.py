"""Terminate legatum-sims servers that are past their expiry tag.

The net under the servers' own `shutdown -h` and the Mac's trap: it runs inside AWS, so a dead
laptop, a failed user-data script or a forgotten batch cannot leave a server running.
"""
import datetime
import os

import boto3

ec2 = boto3.client("ec2")
GRACE = datetime.timedelta(minutes=int(os.environ["GRACE_MINUTES"]))
# Servers with no readable expiry tag are cut off this long after launch.
FALLBACK = datetime.timedelta(hours=int(os.environ["FALLBACK_HOURS"]))


def deadline(instance):
    tags = {t["Key"]: t["Value"] for t in instance.get("Tags", [])}
    try:
        expiry = datetime.datetime.strptime(tags["expiry"], "%Y-%m-%dT%H:%M:%SZ")
        return expiry.replace(tzinfo=datetime.timezone.utc) + GRACE
    except (KeyError, ValueError):
        return instance["LaunchTime"] + FALLBACK


def handler(event, context):
    now = datetime.datetime.now(datetime.timezone.utc)
    doomed = []
    pages = ec2.get_paginator("describe_instances").paginate(
        Filters=[
            {"Name": "tag:project", "Values": ["legatum-sims"]},
            {"Name": "instance-state-name", "Values": ["pending", "running", "stopping", "stopped"]},
        ]
    )
    for page in pages:
        for reservation in page["Reservations"]:
            for instance in reservation["Instances"]:
                if now > deadline(instance):
                    doomed.append(instance["InstanceId"])
    if doomed:
        print("terminating expired servers:", doomed)
        ec2.terminate_instances(InstanceIds=doomed)
    return {"terminated": doomed}
