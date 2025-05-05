#!/bin/bash

echo "🔍 Checking progress files for duplication..."

# Extract seeds_tried from all progress_*.json files
declare -A seed_counts

for file in progress_*.json; do
  if [ -f "$file" ]; then
    count=$(jq -r '.seeds_tried' "$file")
    echo "$file: $count seeds_tried"
    if [[ -n "${seed_counts[$count]}" ]]; then
      echo "⚠️  Duplicate seed count detected: $file and ${seed_counts[$count]} both have $count"
    else
      seed_counts[$count]=$file
    fi
  fi
done

echo "✅ Check complete."
