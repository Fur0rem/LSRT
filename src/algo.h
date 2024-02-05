#ifndef ALGO_H
#define ALGO_H

#include "matrix.h"
#include "link_stream.h"

err_code temporal_floyd_warshall(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma);
/*
    lst & dma -> initialized & not null 

    sets dma to the return value of a "temporal_floyd_warshall"
    executed on lst. 
    see documentation of temporal_floyd_warshall for more informations 
    (documentation not written yet)
*/
#endif 
