#!/bin/bash

make rust_aggressive

for city in data/*; do
    if [ -d "$city" ]; then
        # If the city is already aggressive, we skip it
        if [[ "$city" != *"__Aggressive" ]]; then
            echo "City: $city"
            new_name="${city}__Aggressive"
            cp -r "$city" "$new_name"
            # For all the attacks in the city, we remove the result file
            attacks=$(ls $new_name | grep -v graph)
            for attack in $attacks; do
                rm "$new_name/$attack/result"
            done
        fi
    fi
done

./launch_backup.sh
