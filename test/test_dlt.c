#include "../src/deleted_links.h"


int main(int argc, char ** argv){

    err_code failure ; 

    declare_dlt(dlt);

    FILE * f = fopen("dlt_test_file", "r");
    read_dlt(f, &dlt);


    write_dlt(stdout, &dlt);

    bool del1, del2, del3 ; 

    is_deleted(&del1, &dlt, 0, 1);
    is_deleted(&del2, &dlt, 2, 3);
    is_deleted(&del3, &dlt, 1, 8);


    printf("del1=%u, del2=%u del3=%u\n", del1, del2, del3);


    fclose(f);
    free_dlt(&dlt);

    return 0 ;
}
