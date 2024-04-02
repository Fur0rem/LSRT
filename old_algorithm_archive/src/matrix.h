#ifndef MATRIX_H 
#define MATRIX_H 

#include "link_stream.h"

typedef struct s_mat_dist{
    uint32_t rows ; 
    uint32_t cols ; 

    uint8_t * values ;
    
}DISTANCE_MATRIX ;
/*
    internal matrix rep ; fairly straightforward
    values is of size rows*cols 
*/ 

typedef struct s_dmatrixes{

    uint32_t nb_matrixes ; //time 
    DISTANCE_MATRIX * matrixes ;

}DISTANCE_MATRIX_ARRAY ; 
#define declare_dma(dma) DISTANCE_MATRIX_ARRAY (dma) = {0, NULL};
/*
    matrix array to store the distances ; 
    dma->matrixes[t].value[i][j] stores the 
    distance between i and j at time t
*/

extern err_code init_dma(DISTANCE_MATRIX_ARRAY * dma, uint32_t nb_mat, uint32_t rows, uint32_t cols);
/*
    dma -> not null 
    nb_mat -> number of matrixes 
    rows -> number of rows of each matrix 
    cols -> number of cols of each matrix

    initialized the matrix at dma with nb_mat matrixes of size rows*cols each.

*/

extern void free_dma(DISTANCE_MATRIX_ARRAY * dma);
/*
    frees dma and each of it's matrixes
*/

extern err_code set_elem_dma(DISTANCE_MATRIX_ARRAY * dma , uint8_t value,  uint32_t index, uint32_t row, uint32_t col);
/*
    dma -> not null & initialized
    set dma->matrixes[index].values[i][j] to value ; checks for wrong args
*/

extern err_code get_elem_dma(DISTANCE_MATRIX_ARRAY * dma , uint8_t * value,  uint32_t index, uint32_t row, uint32_t col);
/*
    dma & value -> not null 
    dma -> initialized
    
    fetches dma->matrixes[index].values[i][j] to value ; checks for wrong args 
*/

#ifdef debug 
err_code fprint_dma(FILE * flux, DISTANCE_MATRIX_ARRAY * dma);
#endif
#endif 
