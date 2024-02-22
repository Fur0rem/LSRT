#/bin/bash 
#shitty ass temporary script ; will have to do a better one at some 
#point
if [[ $# -lt 5 ]]; then 
    echo "usage : gen_attack.sh [city name] [attack_type] [nb_attacks] [delta] [budget_stop]" 
    exit 1 
fi

budget_step=100
budget_cur=100

output_name="$1_$2_$3_$4_$5"

if [[ $# -gt 5 ]]; then 
    budget_step=$6
fi 

while [[ budget_cur -le $5 ]]; do 
    python Strategies.py "$1" "$2" "$3" "$4" "$budget_cur"
    
    budget_cur=$((budget_cur + budget_step))

    ./lsrt "$1_graph" "$1_$2_$3_$4" -f "$output_name"  
done
