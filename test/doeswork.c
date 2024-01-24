#include <stdio.h>
#include <stdlib.h>

int main() {
	int* thing = (int*)malloc(sizeof(int));
	*thing = 4;
	printf("thing = %d\n", *thing);
	free(thing);
	return 0;
}