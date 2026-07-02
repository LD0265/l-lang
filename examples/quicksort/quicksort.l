void printarr_i32(i32% arr, i32 len)
{
    i32 i = 0;
    while (i < len) {
        println_i32(arr[i]);
        i += 1;
    }
    return;
}

void swap(i32% a, i32% b)
{
    i32 t = %a;
    %a = %b;
    %b = t;
    return;
}

i32 partition(i32% arr, i32 low, i32 high)
{    
    i32 pivot = arr[high];
    i32 i = low - 1;

    i32 j = low;
    while (j <= high) {
        if (arr[j] < pivot) {
            i += 1;
            swap(&arr[i], &arr[j]);
        }

        j += 1;
    }

    swap(&arr[i + 1], &arr[high]);  
    return i + 1;
}

void quick_sort(i32% arr, i32 low, i32 high)
{
    if (low < high) {
        i32 pi = partition(arr, low, high);

        quick_sort(arr, low, pi - 1);
        quick_sort(arr, pi + 1, high);
    }

    return;
}

void main()
{
    i32% arr = {7, 3, 4, 2, 9, 8, 0, 1, 5, 6};
    quick_sort(arr, 0, 9);
    printarr_i32(arr, 10);
    return;
}