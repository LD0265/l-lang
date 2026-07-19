i32 rand_range(i32 upper) #[no_stack]
{
    __asm__ {
        move    $a1, $a0
        li      $v0, 42
        syscall
        move    $v0, $a0
    }
    return;
}

i32 rand_range_incl(i32 lower, i32 upper)
{
    return rand_range(upper - lower + 1) + lower;
}