#ifndef MATRIX_H 
#define MATRIX_H 

#include "link_stream.h"

typedef struct s_mat_dist{
    uint32_t rows ; 
    uint32_t cols ; 

    uint8_t * values ;
    
}DISTANCE_MATRIX ; 


typedef struct s_dmatrixes{

    uint32_t nb_matrixes ; //time 
    DISTANCE_MATRIX * matrixes ;

}DISTANCE_MATRIX_ARRAY ; 
#define declare_dma(dma) DISTANCE_MATRIX_ARRAY (dma) = {0, NULL};

extern err_code init_dma(DISTANCE_MATRIX_ARRAY * dma, uint32_t nb_mat, uint32_t rows, uint32_t cols);
extern void free_dma(DISTANCE_MATRIX_ARRAY * dma);

extern err_code set_elem_dma(DISTANCE_MATRIX_ARRAY * dma , uint8_t value,  uint32_t index, uint32_t row, uint32_t col);
extern err_code get_elem_dma(DISTANCE_MATRIX_ARRAY * dma , uint8_t * value,  uint32_t index, uint32_t row, uint32_t col);


#ifdef debug 

err_code fprint_dma(FILE * flux, DISTANCE_MATRIX_ARRAY * dma);

#endif

#endif 


