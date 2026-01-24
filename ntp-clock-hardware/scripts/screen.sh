#!/bin/bash


while true; do
    if [ "$(find /dev/ -name 'tty.usbmodem*' 2>/dev/null | wc -l)" -eq 0 ]; then
        echo "Waiting for device..."
        sleep 1
        continue
    else
        break
    fi
done

screen /dev/tty.usbmodem* 115200
reset
exit 0