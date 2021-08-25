#!/bin/bash
set -e

wget https://ci.betrusted.io/latest-ci/loader.bin -O /tmp/loader.bin
./usb_update.py -s /tmp/loader.bin
rm /tmp/loader.bin

echo "waiting for device to reboot"
sleep 5

wget https://ci.betrusted.io/latest-ci/xous.img -O /tmp/xous.img
./usb_update.py -k /tmp/xous.img
rm /tmp/xous.img

echo "waiting for device to reboot"
sleep 5

wget https://ci.betrusted.io/latest-ci/soc_csr.bin -O /tmp/soc_csr.bin
./usb_update.py -s /tmp/soc_csr.bin
rm /tmp/soc_csr.bin

echo "waiting for device to reboot"
sleep 5

wget https://ci.betrusted.io/latest-ci/bt-ec.bin -O /tmp/bt-ec.bin
./usb_update.py -e /tmp/bt-ec.bin
rm /tmp/bt-ec.bin