chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	sub sp, sp, #176
	stp x29, x30, [sp, #80]
	stp x28, x27, [sp, #96]
	stp x26, x25, [sp, #112]
	stp x24, x23, [sp, #128]
	stp x22, x21, [sp, #144]
	stp x20, x19, [sp, #160]
	add x29, sp, #80
	str xzr, [sp]
	adrp x25, :got:chenal::blocking::PARK_WAKER
	mov x21, x3
	ldr x25, [x25, :got_lo12:chenal::blocking::PARK_WAKER]
	mov x23, x2
	mov x22, x1
	mov x19, x0
	mov w0, wzr
	add x20, x1, #256
	mov x26, sp
	mov w27, #-2
	mov w28, #2
.LBB11_1:
	mov x8, x23
	b .LBB11_3
.LBB11_2:
	ldr x8, [x22, #128]
	cmp x8, x23
	b.eq .LBB11_7
.LBB11_3:
	ldr x9, [x22, #456]
	ldr x10, [x22, #440]
	mov x23, x8
	and x8, x9, x8
	add x9, x10, x8, lsl #4
	add x10, x9, #8
	ldar x10, [x10]
	cmp x10, x23
	b.ne .LBB11_2
	ldr x24, [x9]
	dmb ishld
	ldr x9, [x22, #448]
	sub x9, x9, #1
	cmp x8, x9
	b.ne .LBB11_6
	ldr x8, [x22, #456]
	orr x8, x23, x8
	add x9, x8, #1
	add x10, x22, #128
	mov x8, x23
	casal x8, x9, [x10]
	cmp x8, x23
	b.ne .LBB11_3
	b .LBB11_16
.LBB11_6:
	add x9, x23, #1
	add x10, x22, #128
	mov x8, x23
	casal x8, x9, [x10]
	cmp x8, x23
	b.ne .LBB11_3
	b .LBB11_16
.LBB11_7:
	ldar x8, [x20]
	cbz x8, .LBB11_10
	tbnz w8, #0, .LBB11_10
	ldr x8, [x22]
	ldr x9, [x22, #456]
	sub x9, x27, x9, lsr #1
	and w8, w8, w9
	cmp x23, x8
	b.eq .LBB11_20
	tbz w0, #0, .LBB11_13
	mov x0, x21
	bl chenal::blocking::Parker::park
	and w8, w0, #0xff
	mov w0, wzr
	cmp w8, #2
	b.eq .LBB11_1
	b .LBB11_21
	ldr x8, [sp]
	cbnz x8, .LBB11_15
	add x8, x22, #312
	stp xzr, xzr, [x26, #8]
	stur xzr, [x26, #24]
	str x8, [sp]
	strb w28, [sp, #40]
	stp x25, x25, [x29, #-32]
	stur xzr, [x29, #-16]
	mov x0, sp
	sub x1, x29, #32
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	b .LBB11_1
.LBB11_16:
	ldar x8, [x20]
	tbz w8, #0, .LBB11_18
	mov x0, x20
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	str x24, [x19, #8]
	ldr x9, [sp]
	strb wzr, [x19]
	cbnz x9, .LBB11_23
.LBB11_19:
	ldp x20, x19, [sp, #160]
	ldp x22, x21, [sp, #144]
	ldp x24, x23, [sp, #128]
	ldp x26, x25, [sp, #112]
	ldp x28, x27, [sp, #96]
	ldp x29, x30, [sp, #80]
	add sp, sp, #176
	ret
.LBB11_20:
	strb wzr, [x19, #1]
	b .LBB11_22
.LBB11_21:
	strb w8, [x19, #1]
.LBB11_22:
	mov w8, #1
	ldr x9, [sp]
	strb w8, [x19]
	cbz x9, .LBB11_19
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB11_19
	b .LBB11_26
.LBB11_26:
	ldr x8, [sp]
	mov x19, x0
	cbnz x8, .LBB11_28
.LBB11_27:
	mov x0, x19
	bl _Unwind_Resume
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB11_27
	bl core::panicking::panic_in_cleanup
