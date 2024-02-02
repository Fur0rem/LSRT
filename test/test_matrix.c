#include "../src/matrix.h"

int main(int argc, char ** argv){

    declare_dma(dma);

    init_dma(&dma, 3, 5 ,4);

    
    for(uint32_t i = 0 ; i < 5 ; i ++){
        for(uint32_t j = 0 ; j < 4 ; j++){
            set_elem_dma(&dma, j*i, 0, i,j );
            set_elem_dma(&dma, j*i+1, 1, i,j );
            set_elem_dma(&dma, j*i+2, 2, i,j );
     
        }
    }
    
    fprint_dma(stdout, &dma);

    free_dma(&dma);

    return 0 ;
}
