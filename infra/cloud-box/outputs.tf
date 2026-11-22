# tools/cloud/box.sh finds the box by its tags; these are for humans.
output "instance_id" {
  value = aws_instance.box.id
}

output "availability_zone" {
  value = var.availability_zone
}

output "data_volume_id" {
  value = aws_ebs_volume.data.id
}

output "instance_type" {
  value = var.instance_type
}

output "market" {
  value = var.use_spot ? "spot (persistent, stops on interruption)" : "on-demand"
}
