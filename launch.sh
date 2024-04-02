#!/bin/bash

# city : name of the city
# attack : type of attack
# times : number of attacks
# delta : TODO je suis pas sur
# budget_min : how much budget we start with
# budget_max : how much budget we stop with
LONG=city:,attack:,times:,delta:,budget_min:,budget_max:,growth_num:,growth_den:
SHORT=c:,a:,t:,d:,n:,x:,g:,r:
OPTS=$(getopt --options $SHORT --longoptions $LONG -- "$@")

# Defaults
# "" means it should always be set
# -1 means it is optionnal and the Python script will generate a default value
# Anything else is a default value
CITY=""
ATTACK=""
TIMES=-1
DELTA=8
BUDGET_MIN=2
BUDGET_MAX=100
# Fraction so it's more modular
GROWTH_NUMERATOR=3
GROWTH_DENOMINATOR=2

eval set -- "$OPTS"

while true; do
    case "$1" in

    -c | --city)
        CITY=$2
        shift 2
        ;;
    -a | --attack)
        ATTACK=$2
        shift 2
        ;;
    -t | --times)
        TIMES=$2
        shift 2
        ;;
    -d | --delta)
        DELTA=$2
        shift 2
        ;;

    -n | --budget_min) 
        BUDGET_MIN=$2
        shift 2
        ;;
    -x | --budget_max) 
        BUDGET_MAX=$2
        shift 2
        ;;

    -g | --growth_num)
        GROWTH_NUMERATOR=$2
        shift 2
        ;;
    -r | --growth_den)
        GROWTH_DENOMINATOR=$2
        shift 2
        ;;

    --)
        shift
        break
        ;;
    *)
        echo "Unexpected option: $1"
        exit 1
        ;;

    esac
done

FILES_DIR_NAME="intermediate_files"

# If $FILES_DIR_NAME doesn't exist, create it
if [ ! -d "$FILES_DIR_NAME" ]; then
    mkdir $FILES_DIR_NAME
fi

# If the city doesn't exist, create it
if [ ! -d "./$FILES_DIR_NAME/$CITY" ]; then
    mkdir ./$FILES_DIR_NAME/$CITY
fi

# Generate a new temporary filename for the attack
FILE_FORMAT=$(mktemp -d ./$FILES_DIR_NAME/$CITY"/"$ATTACK"_XXXXXX")
if [ $? -ne 0 ]; then
    echo "$0: Can't create temp file"
    exit 1
fi

cur_budg=$BUDGET_MIN
attack_string=""

# Like a pool thread that limits the number of subprocesses
function max_n {
    NB_MAX=4
    while [ `jobs | wc -l` -ge $NB_MAX ]; do
        sleep 1
    done
}

i=0
while [[ $cur_budg -le $BUDGET_MAX ]]; do
    max_n; python3 scripts/Strategies.py "$CITY" "$ATTACK" "$TIMES" "$cur_budg" "$DELTA" "$FILE_FORMAT"/$cur_budg &

    attack_string="$attack_string $cur_budg"
    cur_budg=$((cur_budg * GROWTH_NUMERATOR))
    cur_budg=$((cur_budg / GROWTH_DENOMINATOR))
    i=$((i + 1))
done

# Wait for all the attacks to finish
wait

#python3 scripts/Strategies.py $CITY $ATTACK $TIMES $DELTA $FILE_FORMAT $attack_string
echo "$CITY"_graph "$FILE_FORMAT" -f "$CITY"_"$ATTACK"_"$TIMES"_"$DELTA"_"$BUDGET_MIN"_"$BUDGET_MAX" "$i $attack_string"
CITY_BASENAME=$(basename "$CITY")
FILE_FORMAT_BASENAME=$(basename "$FILE_FORMAT")
./lsrt "$CITY_BASENAME" "$FILE_FORMAT_BASENAME" "$attack_string" 
