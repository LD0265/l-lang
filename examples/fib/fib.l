void main()
{
    i32 a;
    i32 b;
    i32 c;

    while (true) {
        a = 0;
        b = 1;

        while (a < 255) {
            println_i32(a);

            c = a + b;
            a = b;
            b = c;
        }
    }

    return;
}