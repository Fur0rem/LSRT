#include <stdio.h>
#include <stdlib.h>

int main() {
	int* thing = (int*)malloc(sizeof(int));
	*thing = 5;
	printf("thing = %d\n", *thing);
	return 0;
}