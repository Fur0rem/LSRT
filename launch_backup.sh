#!/bin/bash
# For all the cities in the data/ directory, for all the attacks in the city, we look through those with an empty result file and we launch the attack again.

cities=$(ls data/)
for city in $cities; do
    echo "City: $city"
    values=$(python3 scripts/BudgetVille.py $city)
    NB_EDGES=$(echo "$values" | tail -n 1)
    NB_NODES=$(echo "$values" | tail -n 2 | head -n 1)
    DIAMETER=$(echo "$values" | tail -n 3 | head -n 1)
    echo "Number of edges: $NB_EDGES"
    echo "Number of nodes: $NB_NODES"
    echo "Diameter: $DIAMETER"
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
done

