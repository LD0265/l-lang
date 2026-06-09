i32 plus1(i32 n)
{
    return n + 1;
}

void main()
{
    i32 a = 41;
    i32 b = plus1(a);
    i16 c = plus1(1) + 2; // warns type mismatch

    return; // all functions must have a return, even void functions
}