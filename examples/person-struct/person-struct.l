struct person
{
    i8% name[32];
    i8 age;
};

// heap-allocated person
person% person_new(i8% name, i8 age)
{
    person% p = malloc(size person);
    strcpyn(p.name, name, 32);
    p.age = age;
    return p;
}

// stack-allocated person
void person_init(person% this, i8% name, i8 age)
{
    strcpyn(this.name, name, 32);
    this.age = age;
    return;
}

void main()
{
    person% kris;
    person  mike;

    kris = person_new("Kris (Heap)", 14);
    person_init(&mike, "Mike (Stack)", 42);

    return;
}