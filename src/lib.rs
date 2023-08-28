pub mod sumcheck_ml;
pub mod sumcheck_pml;
pub mod sumcheck_naive;

#[cfg(test)]
mod tests {}

pub fn sort_arr<T:Ord + std::marker::Copy>(arr:&mut [T]){
    sorting::merge_sort(arr);
}
pub mod sorting{

    pub fn merge_sort<T:Ord + std::marker::Copy>(array: &mut [T]) {
        let middle = array.len() / 2;
        if array.len() < 2 {
            return; // No need to sort vectors with one element
        }
        
        let mut sorted = array.to_vec();
        
        merge_sort(&mut array[..middle]);
        merge_sort(&mut array[middle..]);
        
        merge(&array[..middle], &array[middle..], &mut sorted);
        
        array.copy_from_slice(&sorted); // Copy the sorted result into original vector
    }
    
    pub fn merge<T:Ord + std::marker::Copy>(l_arr: &[T], r_arr: &[T], sorted: &mut [T]) {
        // Current loop position in left half, right half, and sorted vector
        let (mut left, mut right, mut i) = (0, 0, 0);
        
        while left < l_arr.len() && right < r_arr.len() {
            if l_arr[left] <= r_arr[right] {
            sorted[i] = l_arr[left];
            i += 1;
            left += 1;
            } else {
            sorted[i] = r_arr[right];
            i += 1;
            right += 1;
            }
        }
        
        if left < l_arr.len() {
            // If there is anything left in the left half append it after sorted members
            sorted[i..].copy_from_slice(&l_arr[left..]);
        }
        
        if right < r_arr.len() {
            // If there is anything left in the right half append it after sorted members
            sorted[i..].copy_from_slice(&r_arr[right..]);
        }
    }

    pub fn quick_sort<T:Ord + std::marker::Copy>(array: &mut [T]) {
        let start = 0;
        let end = array.len() - 1;
        quick_sort_partition(array, start, end as isize);
      }
      
      pub fn quick_sort_partition<T:Ord + std::marker::Copy>(array: &mut [T], start: isize, end: isize) {
        if start < end && end - start >= 1 {
          let pivot = partition(array, start as isize, end as isize);
          quick_sort_partition(array, start, pivot - 1);
          quick_sort_partition(array, pivot + 1, end);
        }
      }
      
      pub fn partition<T:Ord + std::marker::Copy>(array: &mut [T], l: isize, h: isize) -> isize {
        let pivot = array[h as usize];
        let mut i = l - 1; // Index of the smaller element
      
        for j in l..h {
          if array[j as usize] <= pivot {
            i = i + 1;
            array.swap(i as usize, j as usize);
          }
        }
      
        array.swap((i + 1) as usize, h as usize);
      
        i + 1
      }
}