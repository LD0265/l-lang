void% malloc(i32 bytes) #[no_stack]
{
    __asm__ {
        li      $v0, 9
        syscall
    }
    return;
}