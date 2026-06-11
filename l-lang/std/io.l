i32 read_i32()
{
    __asm__ {
        li      $v0, 5
        syscall
    }
    return;
}

void print_i32(i32 n)
{
    __asm__ {
        li      $v0, 1
        syscall
    }
    return;
}

void println_i32(i32 n)
{
    __asm__ {
        li      $v0, 1
        syscall
        li      $a0, 10
        li      $v0, 11
        syscall
    }
    return;
}