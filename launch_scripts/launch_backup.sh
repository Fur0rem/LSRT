#!/bin/bash
# For all the cities in the data/ directory, for all the attacks in the city, we look through those with an empty result file and we launch the attack again.

# If there is only one argument, we only run the backup attack on the given city
if [ $# -eq 1 ]; then
    city=$1
    if [ -d "data/$city" ]; then
        echo "City: $city"
        # Remove "graph" file
        attacks=$(ls data/$city/ | grep -v graph)
        for attack in $attacks; do
            if [ ! -s data/$city/$attack/result ]; then
                echo "Attack: $attack"
                # All of them into a single line, except the result file, which is why we use tr with \r.
                # Which is a whitespace character for rust without being an argument separator for bash
                budgets=$(ls data/$city/$attack/ | grep -v result | sort -n | tr '\n' '\r')
                budget_string=""
                for budget in $budgets; do
                    budget_string="$budget_string $budget"
                done
                ./lsrt $city $attack $budget_string
            fi
        done
    else
        echo "City $city does not exist"
    fi
    exit 0
fi

# Otherwise, we run the backup attack on all the cities
cities=$(ls data/)
for city in $cities; do
    echo "City: $city"
    # Remove "graph" file
    attacks=$(ls data/$city/ | grep -v graph)

    # If graph file does not exist or is empty,
    if [ ! -s data/$city/graph ]; then
        echo "Graph file does not exist or is empty"
        exit 1
    fi
    for attack in $attacks; do
        if [ ! -s data/$city/$attack/result ]; then
            echo "Attack: $attack"
            # All of them into a single line, except the result file, which is why we use tr with \r.
            # Which is a whitespace character for rust without being an argument separator for bash
            budgets=$(ls data/$city/$attack/ | grep -v result | sort -n | tr '\n' '\r')
            budget_string=""
            for budget in $budgets; do
                budget_string="$budget_string $budget"
            done
            ./lsrt $city $attack $budget_string
        fi
    done
done

