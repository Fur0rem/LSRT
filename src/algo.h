#ifndef ALGO_H
#define ALGO_H

#include "matrix.h"
#include "link_stream.h"

err_code temporal_floy_warshall(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma);
/*
    oh boy it's time to write code that actually does stuff 

    O(n**3) hopefully 
*/
#endif 
