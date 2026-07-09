i32 read_i32() #[no_stack]
{
    __asm__ {
        li      $v0, 5
        syscall
    }
    return;
}

void read_str(i8% buff, i32 max_chars) #[no_stack]
{
    __asm__ {
        li      $v0, 8
        syscall
    }
    return;
}

void println() #[no_stack]
{
    __asm__ {
        li      $a0, 10
        li      $v0, 11
        syscall
    }
    return;
}

void print_i32(i32 n) #[no_stack]
{
    __asm__ {
        li      $v0, 1
        syscall
    }
    return;
}

void println_i32(i32 n) #[no_stack]
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

void print_str(i8% s) #[no_stack]
{
    __asm__ {
        li      $v0, 4
        syscall
    }
    return;
}

void println_str(i8% s) #[no_stack]
{
    __asm__ {
        li      $v0, 4
        syscall
        li      $a0, 10
        li      $v0, 11
        syscall
    }
    return;
}