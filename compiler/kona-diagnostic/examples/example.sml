(* Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
   root for license information. *)

fun fib n =
   case n in 0 => 0
           | 1 => 1
           | _ => fib (n - 1) + fib (n - 2)
