#ifndef ALGO_H
#define ALGO_H

#include "matrix.h"
#include "link_stream.h"

extern err_code temporal_floyd_warshall(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma);
/*
    lst & dma -> initialized & not null 

    sets dma to the return value of a "temporal_floyd_warshall"
    executed on lst. 
    see documentation of temporal_floyd_warshall for more informations 
    (documentation not written yet&)
*/
extern err_code sum_dma(double * ret_sum, uint64_t * ret_reachables,  DISTANCE_MATRIX_ARRAY * dma);
/*
    ret & dma -> not null

    sums the inverse of the distance of the elements in dma's matrixes 
    and sets *ret to this sum.
*/

err_code prepare_tfw(LINK_STREAM * lst, DISTANCE_MATRIX_ARRAY * dma);
#endif
