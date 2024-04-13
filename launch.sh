#!/bin/bash
# Takes 2 arguments : city and budget_max

# Parameters
# TODO: Change the number of random attacks
NB_ATTACK_RANDOMS=5

# Get the number of edges of the city through the exit value of BudgetVille.py
NB_EDGES=$(python3 scripts/BudgetVille.py $1)

# Launch the non-attacked city
./launch_single.sh -c $1 -a none -g 8 -r 7 -n 0 -x 0 -s 1

# Launch sorted, deterministic attacks
./launch_single.sh -c $1 -a betweenness -g 8 -r 7 -x $NB_EDGES -s 1
./launch_single.sh -c $1 -a betweenness_moving -g 8 -r 7 -x $NB_EDGES -s 1
./launch_single.sh -c $1 -a identifier -g 8 -r 7 -x $NB_EDGES -s 1
./launch_single.sh -c $1 -a longitude -g 8 -r 7 -x $NB_EDGES -s 1
./launch_single.sh -c $1 -a max_lanes -g 8 -r 7 -x $NB_EDGES -s 1

# Launch random attacks
for i in $(seq 1 $NB_ATTACK_RANDOMS); do
    ./launch_single.sh -c $1 -a random -g 8 -r 7 -x $NB_EDGES -s 1
    ./launch_single.sh -c $1 -a random_moving -g 8 -r 7 -x $NB_EDGES -s 1
done

# Exemple :
# ./launch.sh Saints (a prit 5 minutes sur mon ordi pour 5 attaques random), 183 liens
# ./launch.sh Maza (a prit 3 minutes sur mon ordi pour 5 attaques random), 34 liens
