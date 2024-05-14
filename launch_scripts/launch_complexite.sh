# Get the number of edges of the city through the last output of BudgetVille.py
values=$(python3 scripts/BudgetVille.py $1)
# 3rd last line is the diameter
# 2nd last line is the number of nodes
# last line is the number of edges
NB_EDGES=$(echo "$values" | tail -n 1)
NB_NODES=$(echo "$values" | tail -n 2 | head -n 1)
#DIAMETER=$(echo "$values" | tail -n 3 | head -n 1)

echo "City: $1"
echo "Number of edges: $NB_EDGES"
echo "Number of nodes: $NB_NODES"
#echo "Diameter: $DIAMETER"

# Launch the non-attacked city
./launch_scripts/launch_single.sh -c $1 -a full_random -n 1 -x 1 -t 1500
./launch_scripts/launch_single.sh -c $1 -a full_random -n $((NB_EDGES/4)) -x $((NB_EDGES/3)) -t 1500
./launch_scripts/launch_single.sh -c $1 -a full_random -n $((NB_EDGES/2)) -x $((2*NB_EDGES/3)) -t 1500
./launch_scripts/launch_single.sh -c $1 -a full_random -n $((3*NB_EDGES/4)) -x $((3*NB_EDGES/4)) -t 1500
./launch_scripts/launch_single.sh -c $1 -a full_random -n $((NB_EDGES-1)) -x $((NB_EDGES-1)) -t 1500