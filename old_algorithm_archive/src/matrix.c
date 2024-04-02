#include "matrix.h"
#include "misc.h"
#include "utils.h"
#include <stdint.h>

static err_code  init_matrix(DISTANCE_MATRIX * mat,  size_t rows, size_t cols) {
    def_err_handler(!mat,"init_matrix", ERR_NULL);

    mat->values = (uint8_t*)calloc(rows * cols , sizeof(uint8_t));
    def_err_handler(!mat->values, "init_matrix", ERR_ALLOC);

	mat->rows = rows;
	mat->cols = cols;

	return ERR_OK;
}

static void free_matrix(DISTANCE_MATRIX * mat) {
	if(mat){
        if(mat->values){
            free(mat->values);
        }
        mat->values = NULL ; 
        mat->cols = mat->rows = 0 ;
    }
}

static err_code get_elem_mat(uint8_t * ret , DISTANCE_MATRIX* mat, size_t row, size_t col) {
    def_err_handler(!(ret && mat), "get_elem_mat", ERR_NULL);
    def_err_handler((row > mat->rows || col > mat->cols), "get_elem_mat", ERR_VAL);

    *ret = mat->values[row * mat->cols + col];

    return ERR_OK;
}

static err_code set_elem_mat( DISTANCE_MATRIX * mat, uint32_t row, uint32_t col, uint8_t val) {
    def_err_handler(!(mat), "set_elem_mat", ERR_NULL);
    def_err_handler((row > mat->rows || col > mat->cols), "set_elem_mat", ERR_VAL);

	mat->values[row * mat->cols + col] = val;

    return ERR_OK;
}

err_code init_dma(DISTANCE_MATRIX_ARRAY * dma, uint32_t nb_mat, uint32_t rows, uint32_t cols){
    def_err_handler(!dma, "init_dma", ERR_NULL);

    dma->matrixes = calloc(nb_mat, sizeof(DISTANCE_MATRIX));
    def_err_handler(!dma->matrixes, "init_dma", ERR_ALLOC);

    err_code failure ; 
    for(uint32_t i = 0 ; i < nb_mat ; i++){

        failure = init_matrix(&dma->matrixes[i], rows, cols );
        def_err_handler(failure ,"init_dma", failure);
    }

    dma->nb_matrixes = nb_mat ; 
    return ERR_OK;
}// tested ; ok

void free_dma(DISTANCE_MATRIX_ARRAY * dma){
    if(dma){
        if(dma->matrixes){
            for(uint32_t i = 0 ; i < dma->nb_matrixes ; i++ ){
                free_matrix(&dma->matrixes[i]);
            }
            free(dma->matrixes);
        }
        dma->matrixes = NULL ; 
        dma->nb_matrixes = 0 ;
    }
}// tested ; ok

err_code set_elem_dma(DISTANCE_MATRIX_ARRAY * dma , uint8_t value,  uint32_t index, uint32_t row, uint32_t col){
    def_err_handler(!dma, "set_elem_dma", ERR_NULL);
    def_err_handler(index > dma->nb_matrixes , "set_elem_dma", ERR_VAL);

    err_code failure = set_elem_mat(&dma->matrixes[index], row, col ,value);
    def_err_handler(failure ,"set_elem_dma", failure);

    return ERR_OK ; 
}// tested ; ok

err_code get_elem_dma(DISTANCE_MATRIX_ARRAY * dma , uint8_t * ret,  uint32_t index, uint32_t row, uint32_t col){
    def_err_handler(!dma, "get_elem_dma", ERR_NULL);
    def_err_handler(index > dma->nb_matrixes , "get_elem_dma", ERR_VAL);

    err_code failure = get_elem_mat(ret, &dma->matrixes[index], row, col);
    def_err_handler(failure ,"get_elem_dma", failure);

    return ERR_OK ;
}//not tested ; should be ok

#ifdef debug
//debug functions ; doesn't check for shit
static void fprint_matrix( FILE * flux, DISTANCE_MATRIX * mat ){

    for(uint32_t i = 0 ; i < mat->cols * mat->rows ; i++){
        fprintf(flux, "%u ", mat->values[i]);
        if(i % (mat->cols) == mat->cols - 1){
            fprintf(flux, "\n");
        }
    }
}

err_code fprint_dma(FILE * flux, DISTANCE_MATRIX_ARRAY * dma){
     fprint_matrix(flux, &dma->matrixes[0]);
    /*for(uint32_t i = 0 ; i < dma->nb_matrixes ; i++){
        fprint_matrix(flux, &dma->matrixes[i]);
        fprintf(flux, "-----------------\n");
    }*/
    return ERR_OK;
}
#endif
