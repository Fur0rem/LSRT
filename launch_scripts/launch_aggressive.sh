#!/bin/bash

make rust_aggressive

# If there is an argument, we only run the aggressive attack on the given city
if [ $# -eq 1 ]; then
    city=$1
    if [ -d "data/$city" ]; then
        echo "City: $city"
        new_name="data/${city}__Aggressive"
        cp -r "data/$city" "$new_name"
        # For all the attacks in the city, we remove the result file
        attacks=$(ls $new_name | grep -v graph)
        for attack in $attacks; do
            rm "$new_name/$attack/result"
        done
    else
        echo "City $city does not exist"
    fi
    echo "Launching aggressive single backup on city: "$city"__Aggressive"
    ./launch_scripts/launch_backup.sh "$city"__Aggressive
    exit 0
fi

# Otherwise, we run the aggressive attack on all the cities
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

./launch_scripts/launch_backup.sh