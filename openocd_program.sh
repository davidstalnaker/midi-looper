#!/bin/sh
if (( $# != 1 )); then
   echo "Usage:"
   echo "$0 <filename of firmware in ELF format>"
   exit 1
fi

if [ ! -f /tmp/itm.fifo ]; then
  mkfifo /tmp/itm.fifo
fi

openocd \
  -f openocd.cfg \
  -c "init" \
  -c "targets" \
  -c "reset halt" \
  -c "tpiu config internal /tmp/itm.fifo uart off 168000000" \
  -c "itm port 0 on" \
  -c "program $1 verify reset"
