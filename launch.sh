#!/bin/bash
# Takes 2 arguments : city and budget_max

# Parameters
# TODO: Change the number of random attacks
NB_ATTACK_RANDOMS=5

# Get the number of edges of the city through the exit value of BudgetVille.py
# python3 scripts/BudgetVille.py $1
# NB_EDGES=$?

# Get the number of edges of the city through the last output of BudgetVille.py
values=$(python3 scripts/BudgetVille.py $1)
# 3rd last line is the diameter
# 2nd last line is the number of nodes
# last line is the number of edges
NB_EDGES=$(echo "$values" | tail -n 1)
NB_NODES=$(echo "$values" | tail -n 2 | head -n 1)
DIAMETER=$(echo "$values" | tail -n 3 | head -n 1)

echo "City: $1"
echo "Number of edges: $NB_EDGES"
echo "Number of nodes: $NB_NODES"
echo "Diameter: $DIAMETER"

# Launch the non-attacked city
./launch_single.sh -c $1 -a none -n 0 -x 0 -t $DIAMETER

# Launch sorted, deterministic attacks
./launch_single.sh -c $1 -a betweenness -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
./launch_single.sh -c $1 -a betweenness_moving -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
./launch_single.sh -c $1 -a identifier -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
./launch_single.sh -c $1 -a longitude -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
./launch_single.sh -c $1 -a longitude_moving -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
./launch_single.sh -c $1 -a max_lanes -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER

# Launch random attacks
for i in $(seq 1 $NB_ATTACK_RANDOMS); do
    ./launch_single.sh -c $1 -a full_random -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
    ./launch_single.sh -c $1 -a random_moving -g 15 -r 14 -x $NB_EDGES -s 2 -t $DIAMETER
done

# Exemple :
# ./launch.sh Saints (a prit 5 minutes sur mon ordi pour 5 attaques random), 183 liens
# ./launch.sh Maza (a prit 3 minutes sur mon ordi pour 5 attaques random), 34 liens
