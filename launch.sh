#!/bin/bash
# Takes 2 arguments : city and budget_max

# Parameters
NB_ATTACK_RANDOMS=5

# Launch random attacks
for i in $(seq 1 $NB_ATTACK_RANDOMS); do
    ./launch_single.sh -c $1 -a random -g 10 -r 9 -x $2 -s 1
    ./launch_single.sh -c $1 -a random_moving -g 10 -r 9 -x $2 -s 1
done

# Launch betweenness attacks
./launch_single.sh -c $1 -a betweenness -g 10 -r 9 -x $2 -s 1
./launch_single.sh -c $1 -a betweenness_moving -g 10 -r 9 -x $2 -s 1

# Launch mix_lanes and max_lanes attacks
./launch_single.sh -c $1 -a min_lanes -g 10 -r 9 -x $2 -s 1
./launch_single.sh -c $1 -a max_lanes -g 10 -r 9 -x $2 -s 1

# Exemple :
# ./launch.sh Saints 183 (a prit 5 minutes sur mon ordi pour 5 attaques random)