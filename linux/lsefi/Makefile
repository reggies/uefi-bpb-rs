OBJ := lsefi.o
HDR := pe.h

%.o: %.c $(HDR)
	gcc -c $< -o $@

all: $(OBJ)
	gcc $(OBJ) -o lsefi
