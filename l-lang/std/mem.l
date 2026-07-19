void% malloc(i32 bytes) #[no_stack]
{
    __asm__ {
        li      $v0, 9
        syscall
    }
    return;
}

void memset(void% ptr, i8 data, i32 size)
{
    i32 i = 0;
    i8% cast = ptr;
    while (i < size) {
        %(cast + i) = data;
        i += 1;
    }
}