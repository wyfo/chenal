mpmc_recv:
	sub sp, sp, #96
	stp x29, x30, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #64
	ldr x8, [x0]
	add x20, sp, #8
	add x0, x8, #128
	stp x0, xzr, [sp, #8]
	ldr x3, [x8, #256]
	ldr x9, [x8, #584]
	ldr x10, [x8, #568]
	and x9, x9, x3
	add x10, x10, x9, lsl #4
	add x11, x10, #8
	ldar x11, [x11]
	cmp x11, x3
	b.ne .LBB14_3
	ldr x19, [x10]
	dmb ishld
	ldr x10, [x8, #576]
	sub x10, x10, #1
	cmp x9, x10
	b.ne .LBB14_6
	ldr x9, [x8, #584]
	orr x9, x3, x9
	add x9, x9, #1
	add x10, x8, #256
	mov x11, x3
	casal x11, x9, [x10]
	cmp x11, x3
	mov x3, x11
	b.eq .LBB14_7
.LBB14_3:
	ldr x1, [x1]
	add x2, x20, #8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	mov w8, #2
	cmp x0, #2
	and x9, x0, #0x1
	csel x19, x8, x1, eq
	csel x0, x8, x9, eq
	ldr x8, [sp, #16]
	cbnz x8, .LBB14_10
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
.LBB14_6:
	add x9, x3, #1
	add x10, x8, #256
	mov x11, x3
	casal x11, x9, [x10]
	cmp x11, x3
	mov x3, x11
	b.ne .LBB14_3
.LBB14_7:
	add x0, x8, #384
	ldar x8, [x0]
	tbz w8, #0, .LBB14_9
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x0, xzr
	ldr x8, [sp, #16]
	cbz x8, .LBB14_5
	add x8, sp, #8
	mov x20, x0
	add x0, x8, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov x0, x20
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
	ldr x8, [sp, #16]
	mov x19, x0
	cbnz x8, .LBB14_13
.LBB14_12:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB14_12
	bl core::panicking::panic_in_cleanup
