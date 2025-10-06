#!/bin/bash
if command -v free &> /dev/null; then
  free -h
else
  vm_stat | perl -ne '/page size of (\d+)/ and $size=$1; /Pages\s+([^:]+)[^\d]+(\d+)/ and printf("%-16s % 16.2f Mi\n", "$1:", $2 * $size / 1048576);'
fi
df -h
docker stats
