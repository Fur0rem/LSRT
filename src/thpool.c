#include "thpool.h"
#include "common.h"
#include "job_deque.h"
#include "utils.h"
#include <pthread.h>

#define DEF_THREAD_COUNT 4
uint32_t global_nb_threads = DEF_THREAD_COUNT;

#define f_running 1
#define f_accepting 2
#define f_shutdown 4

/* STATIC THTAB MANIPULATION */
static err_code init_thtab(S_THTAB * thtab){
    def_err_handler(!thtab, "thtab_init", ERR_NULL);

    thtab->threads = (pthread_t *)calloc(global_nb_threads, sizeof(pthread_t));
    def_err_handler(!thtab->threads, "thtab_init", ERR_ALLOC)

    thtab->size =global_nb_threads;
    return ERR_OK;
}//yeah

static void free_thtab(S_THTAB * thtab){
    if(thtab){
        if(thtab->threads){
            free(thtab->threads); 
        }
        thtab->threads = NULL ; 
        thtab->size = 0 ; 
    }
}//yeah

/* THPOOL PRIMITIVES */
err_code thpool_init(S_THPOOL * pool ){
    def_err_handler(!pool, "thpool_init", ERR_NULL);

    err_code failure = init_thtab(&pool->thtab);
    def_err_handler(failure, "thpool_init", failure);

    failure = task_queue_init(&pool->task_queue);
    def_err_handler(failure, "thpool_init", failure);

    pool->flags = f_running | f_accepting;
    pool->nb_th_working = 0;

    pool->nb_submitted = pool->nb_completed = 0;

    pthread_mutex_init(&pool->mutex_queue, NULL);
    pthread_cond_init(&pool->cond_queue, NULL);

    pthread_mutex_init(&pool->mutex_sub_equals_comp, NULL);
    pthread_cond_init(&pool->cond_sub_equals_comp, NULL);

    return ERR_OK ;
}


static void * thpool_worker(void * arg) ; 

err_code thpool_start(S_THPOOL *pool){
    def_err_handler(!pool, "thpool_start", ERR_NULL);

    for(uint32_t i = 0 ; i < pool->thtab.size ; i++){
        int err = pthread_create(&pool->thtab.threads[i],
             NULL, thpool_worker, pool);
        def_err_handler(err, "thpool_start", ERR_THSTART);
    }
    return ERR_OK;
}

err_code thpool_append_task(S_THPOOL *pool, S_TASK *task){
    def_err_handler(!pool, "thpool_append_task", ERR_NULL);
    def_err_handler(!task, "thpool_append_task", ERR_NULL);
    
    if((! (pool->flags & f_accepting) )) return ERR_OK;
    
    pthread_mutex_lock(&pool->mutex_queue);
    pool->nb_submitted ++ ; 
    err_code failure = task_queue_push(&pool->task_queue, task);
    def_err_handler(failure, "thpool_append_task", failure);
    
    pthread_mutex_unlock(&pool->mutex_queue);
    pthread_cond_signal(&pool->cond_queue);
    
    return ERR_OK;
}


err_code thpool_append_task_batch(S_THPOOL *pool, S_TASK *tasks, uint32_t nb_tasks){
    def_err_handler(!pool, "thpool_append_tasks", ERR_NULL);
    def_err_handler(!tasks, "thpool_append_tasks", ERR_NULL);
    if((! (pool->flags & f_accepting) )) return ERR_OK;
    
    pthread_mutex_lock(&pool->mutex_queue);
    for(uint32_t i = 0 ; i < nb_tasks ; i++){
        err_code failure = task_queue_push(&pool->task_queue, &tasks[i]);
        def_err_handler(failure, "thpool_append_task", failure);
    }
    pthread_mutex_unlock(&pool->mutex_queue);
    pthread_cond_signal(&pool->cond_queue);

    return ERR_OK;
}


err_code thpool_wait_for_all(S_THPOOL * pool){
    def_err_handler(!pool, "thpool_wait_for_all", ERR_NULL);
    pool->flags |= !f_accepting;

    pthread_mutex_lock(&pool->mutex_sub_equals_comp);
    while( pool->nb_completed < pool->nb_submitted){
        pthread_cond_wait(&pool->cond_sub_equals_comp, &pool->mutex_sub_equals_comp);
    }
    pthread_mutex_unlock(&pool->mutex_sub_equals_comp);

    return ERR_OK ;
}

err_code thpool_restart(S_THPOOL * pool){
    def_err_handler(!pool, "thpool_restart", ERR_NULL); 

    pthread_mutex_lock(&pool->mutex_sub_equals_comp);
    while( pool->nb_completed < pool->nb_submitted){
        pthread_cond_wait(&pool->cond_sub_equals_comp, &pool->mutex_sub_equals_comp);
    }
    pthread_mutex_unlock(&pool->mutex_sub_equals_comp);
    pool->flags |= f_accepting;
    //pool->nb_submitted = pool->nb_completed = 0 ;

    return ERR_OK;
}

err_code thpool_destroy(S_THPOOL *pool){
    def_err_handler(!pool, "thpool_destroy", ERR_NULL);

    thpool_wait_for_all(pool);  
    pool->flags |= f_shutdown;

    pthread_cond_broadcast(&pool->cond_queue);
    for(uint32_t i = 0 ; i < pool->thtab.size ; i++){
        pthread_join(pool->thtab.threads[i], NULL);
    }

    free_thtab(&pool->thtab);
    task_queue_destroy(&pool->task_queue);
    pthread_mutex_destroy(&pool->mutex_queue);
    pthread_cond_destroy(&pool->cond_queue);

    return ERR_OK;
}

/* THE ACTUAL WORKER FUNCTION */

static void * thpool_worker(void * arg){
    S_THPOOL * pool = (S_THPOOL *) arg;
    S_TASK task;

    while(1){
        
        pthread_mutex_lock(&pool->mutex_queue);
        while(! pool->task_queue.size && !(pool->flags & f_shutdown) ){
            pthread_cond_wait(&pool->cond_queue, &pool->mutex_queue);
        }

        if(pool->flags & f_shutdown){
            pthread_mutex_unlock(&pool->mutex_queue);
            pthread_exit(NULL);
        }

        task_queue_pop(&pool->task_queue, &task);
        pool->nb_th_working++;
        pthread_mutex_unlock(&pool->mutex_queue);
      
        if(!task.function){
            //unrecoverable error
            def_war_handler(ERR_VAL, "thpool_worker", ERR_VAL);
            pthread_exit(NULL);      
        }else{ 
            task.function(task.args);
            pool->nb_completed++;
        }
      
        if(pool->nb_completed <= pool->nb_submitted){
            pthread_cond_signal(&pool->cond_sub_equals_comp);
        }          
        pool->nb_th_working--;
    }
    return NULL;      
}
