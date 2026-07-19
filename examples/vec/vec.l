struct vec
{
    i32% data;
    i32 len;
    i32 cap;
};

void vec_init(vec% v)
{
    v.data = malloc(sizeof i32);
    v.len = 0;
    v.cap = 1;
}

void push(vec% v, i32 n)
{
    if (v.len == v.cap) {
        i32 new_cap = v.cap * 2;
        i32% new_data = malloc(sizeof i32 * new_cap);

        for (i32 i = 0; i < v.cap; i++) {
            new_data[i] = v.data[i];
        }

        v.data = new_data;
        v.cap = new_cap;
    }

    v.data[v.len] = n;
    v.len = v.len + 1;
}

void print_vec(vec% v)
{
    print_str("[");

    if (v.len > 0) {
        for (i32 i = 0; i < v.len - 1; i++) {
            printf("%d, ", v.data[i]);
        }

        printf("%d", v.data[v.len - 1]);
    }

    print_str("]\n");
}

void main()
{
    vec v;

    vec_init(&v);

    push(&v, 1);
    push(&v, 2);
    push(&v, 3);
    push(&v, 4);

    print_vec(&v);

    printf("Vec len: %d\n", v.len);
    printf("Vec cap: %d\n", v.cap);
}