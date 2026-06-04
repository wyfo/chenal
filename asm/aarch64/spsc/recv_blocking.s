spsc_recv_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	str x19, [sp, #64]
	add x29, sp, #48
	ldr x19, [x0]
	ldr x2, [x19, #256]
	mov w8, w2
	cmp x8, x2, lsr #32
	b.eq .LBB2_4
	str x2, [sp, #8]
.LBB2_2:
	ldr x8, [sp, #8]
	ldr x9, [x19, #544]
	ldr x10, [x19, #536]
	and x11, x9, x8
	orr w9, w8, w9
	sub x10, x10, #1
	add w9, w9, #1
	and x12, x8, #0xffffffff00000000
	cmp x11, x10
	ldr x10, [x19, #528]
	orr x9, x9, x12
	csinc x8, x9, x8, eq
	ldr x1, [x10, x11, lsl #3]
	str x8, [x19, #256]
	add x8, x19, #416
	dmb ish
	ldar x8, [x8]
	cmp x8, #1
	b.ls .LBB2_6
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB2_4:
	mov w8, #51712
	mov x0, sp
	add x1, x19, #128
	movk w8, #15258, lsl #16
	add x3, sp, #16
	str w8, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	cmp w8, #1
	b.ne .LBB2_2
	mov w0, #1
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB2_6:
	add x0, x19, #384
	mov x19, x1
	mov x1, x8
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
	mov x1, x19
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
