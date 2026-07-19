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

void print_i16(i16 n) #[no_stack]
{
    __asm__ {
        li      $v0, 1
        syscall
    }
    return;
}

void println_i16(i16 n) #[no_stack]
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

void print_char(i8 c) #[no_stack]
{
    __asm__ {
        li      $v0, 11
        syscall
    }

    return;
}

void print_ptr(void% p) #[no_stack]
{
    __asm__ {
        li      $v0, 34
        syscall
    }
    return;
}

void println_ptr(void% p) #[no_stack]
{
    __asm__ {
        li      $v0, 34
        syscall
        li      $a0, 10
        li      $v0, 11
        syscall
    }
    return;
}

void printf(i8% fmt, void arg)
{
    while (%fmt != 0) {
        if (%fmt == 37) {
            fmt += 1;

            if (%fmt == 100) {
                i32 iarg = arg;
                print_i32(iarg);
            } else {
                if (%fmt == 115) {
                    i8% sarg = arg;
                    print_str(sarg);
                } else {
                    if (%fmt == 99) {
                        i8 carg = arg;
                        print_char(carg);
                    } else {
                        if (%fmt == 112) {
                            void% parg = arg;
                            print_ptr(parg);
                        } else {
                            if (%fmt == 37) {
                                i8 percent = 37;
                                print_char(percent);
                            }
                        }
                    }
                }
            }
        } else {
            print_char(%fmt);
        }

        fmt += 1;
    }

    return;
}