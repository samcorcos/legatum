#!/bin/bash
# Run as root on the box, over ssh stdin, before every script sync (idempotent). First-boot user-data can run before
# Terraform has attached the data volume (the attachment completes after the instance boots), leaving /data on the
# root disk. This formats (once) and mounts the volume, moving anything already under /data onto it.
set -eu
mountpoint -q /data && exit 0
root_src="$(findmnt -no SOURCE /)"
root_disk="$(lsblk -no PKNAME "$root_src" | head -1)"
dev=""
for d in $(lsblk -dn -o NAME,TYPE | awk '$2=="disk"{print $1}'); do
  [ "$d" != "$root_disk" ] || continue
  # an unmounted whole disk of at least 100 GB with no partitions
  [ "$(lsblk -n -o NAME "/dev/$d" | wc -l)" = 1 ] || continue
  [ "$(lsblk -bdn -o SIZE "/dev/$d")" -ge 107374182400 ] || continue
  dev="/dev/$d"; break
done
[ -n "$dev" ] || { echo "mount-data: no data volume found yet" >&2; exit 1; }
command -v mkfs.xfs >/dev/null || { DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=300 install -y -qq xfsprogs; }
blkid "$dev" >/dev/null 2>&1 || mkfs.xfs -f -L legatum-data "$dev"
mkdir -p /mnt/newdata
mount "$dev" /mnt/newdata
[ ! -d /data ] || cp -a /data/. /mnt/newdata/ 2>/dev/null || true
umount /mnt/newdata
rm -rf /data/* /data/.[!.]* 2>/dev/null || true
mkdir -p /data
grep -q ' /data ' /etc/fstab || echo "LABEL=legatum-data /data xfs defaults,noatime,nofail 0 2" >> /etc/fstab
mount /data
chown ubuntu:ubuntu /data
echo "mount-data: mounted $dev at /data"
