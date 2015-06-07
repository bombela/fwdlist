#  fwdlist - [documentation reference](http://bombela.github.io/fwdlist/fwdlist/struct.List.html)

[![Build Status](https://travis-ci.org/bombela/fwdlist.svg?branch=master)](https://travis-ci.org/bombela/fwdlist)

A simple forward linked list.

It's a linked list. Its not cache friendly, its relatively slow when you think
about it, but it allows for O(1) insertion... after the current iterator
location, maybe you care about that.

# Avoiding unsafe
The goal here is to play with Rust and see how much unsafe is needed. It turns
out that you can implement everything but the mutable iterator without using
unsafe.

The mutable iterator needs unsafe only because it returns a mutable reference
with a different lifetime than the mutable reference on the iterator itself. The
compiler cannot infer that auto-magically and needs a bit of our help.

# penultimate_link() performances

Sometimes the code feels a more convoluted than necessary to please the borrow
checker.  Some unsafe code would make the code not only easier to read, but also
*we naively believe*, more efficient for the machine.

The best example here is `penultimate_link()`, which returns a mutable reference
to the last but one link of the list.

To illustrate what this function returns, let's assume the following list:

```text
head_link -> node1.next -> node2.next -> node3.next -> nil
```

In this case, `penultimate_link()` will return a mutable reference to
`node2.next`. It is then trivial to implement `pop_back()` with a simple
`Option.take()`.

See `penultimate_link()` and `penultimate_link_with_unsafe()` implementations
further below.

## Assembly output

Take a look at the assembly outputs (cargo build --release) below:

* `penultimate_link()`:

```gas
0000000000016200 <::only_safe::>:
   16200:	48 8b 4f 08          	mov    0x8(%rdi),%rcx
   16204:	31 c0                	xor    %eax,%eax
   16206:	48 85 c9             	test   %rcx,%rcx
   16209:	74 1f                	je     1622a <::only_safe::+0x2a>
   1620b:	31 c0                	xor    %eax,%eax
   1620d:	0f 1f 00             	nopl   (%rax)
   16210:	48 89 ca             	mov    %rcx,%rdx
   16213:	48 8b 4a 08          	mov    0x8(%rdx),%rcx
   16217:	48 85 c9             	test   %rcx,%rcx
   1621a:	74 0e                	je     1622a <::only_safe::+0x2a>
   1621c:	48 83 79 08 00       	cmpq   $0x0,0x8(%rcx)
   16221:	75 ed                	jne    16210 <::only_safe::+0x10>
   16223:	48 83 c2 08          	add    $0x8,%rdx
   16227:	48 89 d0             	mov    %rdx,%rax
   1622a:	c3                   	retq
```
* `penultimate_link_with_unsafe()`:

```gas
00000000000168a0 <::with_unsafe::>:
   168a0:	31 c0                	xor    %eax,%eax
   168a2:	48 83 7f 08 00       	cmpq   $0x0,0x8(%rdi)
   168a7:	74 18                	je     168c1 <::with_unsafe::+0x21>
   168a9:	48 83 c7 08          	add    $0x8,%rdi
   168ad:	0f 1f 00             	nopl   (%rax)
   168b0:	48 8b 0f             	mov    (%rdi),%rcx
   168b3:	48 83 79 08 00       	cmpq   $0x0,0x8(%rcx)
   168b8:	48 89 f8             	mov    %rdi,%rax
   168bb:	48 8d 79 08          	lea    0x8(%rcx),%rdi
   168bf:	75 ef                	jne    168b0 <::with_unsafe::+0x10>
   168c1:	c3                   	retq
```
## Assembly quick analysis

The first thing to note, is how well the original code is translated from high
level Option and Box to simple null-able pointers.

* `penultimate_link()` is a loop with two conditional branches inside, and it
  tests twice every nodes of the list (exactly like in the Rust code). One test
on every next_link, before testing it again when it become the new link to work
on new every new iteration.
* `penultimate_with_unsafe()` is a loop with only one condition, but it keeps a
  “prev_link” pointer handy, again like in the Rust code.

Looking at the assembly with my ridiculously weak knowledge of modern CPU
architecture, I infer that `penultimate_link()` requires twice the amount of
branches predictions and both functions perform two data read per iteration.

Considering how modern CPUs seems to pipeline/pre-fetch like crazy, the two
branchs predictions should pretty much cost like only one.

## Callgrind/Cachegrind (valgrind) analysis

After adding `#[inline(never)]` on both `penultimate_link*` functions, I ran
valgrind like so:

```sh
$ valgrind --tool=callgrind --dump-instr=yes --trace-jump=yes --cache-sim=yes --branch-sim=yes --collect-atstart=no --toggle-collect=*penultimate_link* target/release/fwdlist... --test one_penultimate
```
We basically get the following report:

| version   | Ir        | Dr        | D1mr      | DLmr    | Bc        | Bcm |
|-----------|-----------|-----------|-----------|---------|-----------|-----|
| safe_only | 6,291,459 | 2,097,152 | 1 261,697 | 236,874 | 2,097,151 | 4   |
| unsafe    | 5,242,886 | 2,097,154 | 1 261,697 | 238,678 | 1,048,577 | 5   |

* **Ir**: instruction read, `penultimate_link()` has more instructions and so
  more instruction read.
* **Dr**: data read. `penultimate_with_unsafe()` performs one more loop
  iteration, reading **2** more data.
* **D1mr**: data read misses on L1 cache. Similar between the two.
* **DLmr**: data read misses on Last Level cache. Interestingly,
  `penultimate_with_unsafe()` has more misses.
* **Bc**: Conditional branches. Confirms that `penultimate_link()` has two vs
  one conditions.
* **Bcm**: Conditional branches misses. `penultimate_with_unsafe()` gets one
  more, maybe the extra iteration?

## Benchmark

`penultimate_link()` is faster than `penultimate_with_unsafe()` on real hardware.

Benchmarks with List\<i64\> and BIGLIST_SIZE=2^20 (list takes ~16Mib):

```text
AMD Phenom(tm) II X4 965 Processor
penultimate_safe        ... bench:   3651099 ns/iter (+/- 35924)
penultimate_with_unsafe ... bench:   3687377 ns/iter (+/- 33386)

Intel(R) Core(TM) i7-2720QM CPU @ 2.20GHz
penultimate_safe        ... bench:   2333951 ns/iter (+/- 27634)
penultimate_with_unsafe ... bench:   2334611 ns/iter (+/- 43642)

Intel(R) Core(TM) i5-3320M CPU @ 2.60GHz
penultimate_safe        ... bench:   1675111 ns/iter (+/- 106477)
penultimate_with_unsafe ... bench:   2127297 ns/iter (+/- 128966)
```

Benchmarks with List\<i64\> and BIGLIST_SIZE=2^30 (list takes ~16Gib):

```text
Intel(R) Xeon(R) CPU E5-1650 0 @ 3.20GHz
penultimate_safe        ... bench: 2399497518 ns/iter (+/- 357540058)
penultimate_with_unsafe ... bench: 2509462341 ns/iter (+/- 377119880)
```
## Performances conclusion

Convoluted safe code vs simpler unsafe code doesn't necessary mean that unsafe
code is going to be faster. In our specific case `penultimate_with_unsafe()` is
indeed slower!

This is great because with safe Rust code only, the compiler basically proves
for us that there is no possible memory bugs. Any code refactoring cannot
possibly introduce memory bugs easier, the compiler wouldn't let it pass.


Happy hacking!
