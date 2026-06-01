spmc_recv_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	str x19, [sp, #64]
	add x29, sp, #48
	ldr x19, [x0]
	add x8, x19, #256
	ldar x8, [x8]
	mov w9, w8
	mov x2, x8
	cmp x9, x8, lsr #32
	b.eq .LBB13_3
	ldr x9, [x19, #560]
	ldr x10, [x19, #544]
	and x11, x9, x8
	ldr x9, [x10, x11, lsl #3]
	dmb ishld
	ldr x10, [x19, #552]
	sub x10, x10, #1
	cmp x11, x10
	b.ne .LBB13_5
	ldr w10, [x19, #560]
	and x11, x8, #0xffffffff00000000
	orr w10, w8, w10
	add w10, w10, #1
	orr x10, x10, x11
	add x11, x19, #256
	casal x2, x10, [x11]
	cmp x2, x8
	b.eq .LBB13_6
.LBB13_3:
	mov w8, #51712
	mov x0, sp
	add x1, x19, #128
	movk w8, #15258, lsl #16
	add x3, sp, #16
	str w8, [sp, #40]
	bl chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	cmp w8, #1
	b.ne .LBB13_7
	mov w0, #1
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB13_5:
	add x10, x8, #1
	add x11, x19, #256
	casal x2, x10, [x11]
	cmp x2, x8
	b.ne .LBB13_3
.LBB13_6:
	str x9, [sp, #8]
.LBB13_7:
	add x8, x19, #416
	ldr x1, [sp, #8]
	ldar x8, [x8]
	cmp x8, #1
	b.ls .LBB13_9
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB13_9:
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
