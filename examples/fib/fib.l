i32 fib(i32 n)
{
    if (n < 2) {
        return n;
    }

    return fib(n - 1) + fib(n - 2);
}

// until we have loops
void for(i32 i, i32 max)
{
    if (i > max) {
        return;
    }
    println_i32(fib(i));
    for(i + 1, max);
    return;
}

void main()
{
    i32 n = read_i32();
    for(0, n);
    return;
}