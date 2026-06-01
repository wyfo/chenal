mpmc_recv:
	sub sp, sp, #96
	stp x29, x30, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #64
	ldr x8, [x0]
	add x0, x8, #128
	stp x0, xzr, [sp, #8]
	ldr x9, [x8, #256]
	ldr x10, [x8, #584]
	ldr x11, [x8, #568]
	and x10, x10, x9
	mov x3, x9
	add x11, x11, x10, lsl #4
	add x12, x11, #8
	ldar x12, [x12]
	cmp x12, x9
	b.ne .LBB12_6
	ldr x19, [x11]
	dmb ishld
	ldr x11, [x8, #576]
	sub x11, x11, #1
	cmp x10, x11
	b.ne .LBB12_3
	ldr x10, [x8, #584]
	orr x10, x9, x10
	add x10, x10, #1
	b .LBB12_4
.LBB12_3:
	add x10, x9, #1
.LBB12_4:
	add x8, x8, #256
	casal x3, x10, [x8]
	cmp x3, x9
	b.ne .LBB12_6
	mov x0, xzr
	b .LBB12_8
.LBB12_6:
	ldr x1, [x1]
	add x20, sp, #8
	add x2, x20, #8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	mov w8, #2
	cmp x0, #2
	and x9, x0, #0x1
	csel x19, x8, x1, eq
	csel x0, x8, x9, eq
.LBB12_8:
	ldr x8, [sp, #16]
	cbnz x8, .LBB12_10
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
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
	cbnz x8, .LBB12_13
.LBB12_12:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB12_12
	bl core::panicking::panic_in_cleanup
