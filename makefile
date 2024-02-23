TARGET := lsrt
DEBUG := lsrt_debug

TEST := test
TEST_DBG := test_dbg

CC := gcc

DEBUG_FLAGS := -g -fsanitize=address -fsanitize=object-size -fno-optimize-sibling-calls -fsanitize=undefined -fsanitize=leak -fsanitize=alignment
CFLAGS := -O2 -std=gnu17 -Wall -Wextra -Wpedantic -Wno-unused-parameter 

SRCS := $(wildcard src/*.c)

all: $(TARGET)

$(TARGET):  $(SRCS) main.c
	$(CC) $(CFLAGS) $^ -o $(TARGET)
$(DEBUG) :   $(SRCS) main.c
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $^ -o $(DEBUG)


test: $(TEST)
$(TEST): $(SRCS) $(wildcard test/*.c)
	for file in $(wildcard test/*.c); do \
		$(CC) $(CFLAGS) $$file $(SRCS) -o test/bin/$$(basename $$file .c); \
	done

clean: 
	rm -f lsrt lsrt_debug test/bin/*

