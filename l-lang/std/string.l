i32 strlen(i8% str)
{
    i8% old = str;
    while (%str) { str += 1; }
    return str - old;
}

void strcpy(i8% dst, i8% src)
{
    i32 i = 0;
    while (src[i]) {
        %(dst + i) = src[i];
        i += 1;
    }
    return;
}

void strcpyn(i8% dst, i8% src, i32 n)
{
    i32 i = 0;

    while (i < n - 1 && src[i] != 0) {
        dst[i] = src[i];
        i += 1;
    }

    if (n > 0) {
        dst[i] = 0;
    }
    
    return;
}