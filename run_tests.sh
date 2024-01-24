error=0
for test in test/bin/*; do
failure=$(valgrind --leak-check=full --error-exitcode=1 --show-leak-kinds=all --errors-for-leak-kinds=all $test 2>&1)
if [ $? -ne 0 ]; then
	error=1
	echo $failure
	echo "Test $test failed"
else
	echo "Test $test passed"
fi
done
exit $error