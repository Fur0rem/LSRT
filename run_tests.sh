error=0
RED='\033[1;31m'
GREEN='\033[1;32m'
NO_COLOR='\033[0m'
for test in test/bin/*; do
failure=$(valgrind -s --leak-check=full --error-exitcode=1 --show-leak-kinds=all --errors-for-leak-kinds=all $test 2>&1)
if [ $? -ne 0 ]; then
	error=1
	echo ''
	echo -e "Test $RED${test##*/}$NO_COLOR failed"
	echo $failure | sed 's/ ==/\n==/g'
	echo ''
else
	echo -e "Test $GREEN${test##*/}$NO_COLOR passed"
fi
done
exit $error