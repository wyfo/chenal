chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	sub sp, sp, #112
	stp x29, x30, [sp, #32]
	str x25, [sp, #48]
	stp x24, x23, [sp, #64]
	stp x22, x21, [sp, #80]
	stp x20, x19, [sp, #96]
	add x29, sp, #32
	mov x20, x0
	mov x23, x3
	mov x21, x2
	mov x22, x1
	mov w0, wzr
	mov w24, #-2
	add x25, x20, #312
	mov w19, #2
.LBB11_1:
	mov x8, x23
	b .LBB11_3
.LBB11_2:
	ldr x8, [x20, #128]
	cmp x8, x23
	b.eq .LBB11_7
.LBB11_3:
	ldr x9, [x20, #456]
	ldr x10, [x20, #440]
	mov x23, x8
	and x8, x9, x8
	add x9, x10, x8, lsl #4
	add x10, x9, #8
	ldar x10, [x10]
	cmp x10, x23
	b.ne .LBB11_2
	ldr x1, [x9]
	dmb ishld
	ldr x9, [x20, #448]
	sub x9, x9, #1
	cmp x8, x9
	b.ne .LBB11_6
	ldr x8, [x20, #456]
	orr x8, x23, x8
	add x9, x8, #1
	add x10, x20, #128
	mov x8, x23
	casal x8, x9, [x10]
	cmp x8, x23
	b.ne .LBB11_3
	b .LBB11_14
.LBB11_6:
	add x9, x23, #1
	add x10, x20, #128
	mov x8, x23
	casal x8, x9, [x10]
	cmp x8, x23
	b.ne .LBB11_3
	b .LBB11_14
.LBB11_7:
	add x8, x20, #256
	ldar x8, [x8]
	cbz x8, .LBB11_10
	tbnz w8, #0, .LBB11_10
	ldr x8, [x20]
	ldr x9, [x20, #456]
	sub x9, x24, x9, lsr #1
	and w8, w8, w9
	cmp x23, x8
	cset w8, eq
	orr w8, w0, w8
	tbz w8, #0, .LBB11_11
	b .LBB11_16
	tbnz w0, #0, .LBB11_17
	ldr x8, [x21]
	cbnz x8, .LBB11_13
	stp x25, xzr, [x21]
	stp xzr, xzr, [x21, #16]
	strb w19, [x21, #40]
	mov x1, sp
	mov x0, x21
	stp x22, x22, [sp]
	str xzr, [sp, #16]
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	b .LBB11_1
.LBB11_14:
	add x0, x20, #256
	ldar x8, [x0]
	tbnz w8, #0, .LBB11_19
	mov x19, xzr
	b .LBB11_18
.LBB11_16:
	mov w8, #1
	cinc x19, x8, ne
.LBB11_18:
	mov x0, x19
	ldp x20, x19, [sp, #96]
	ldr x25, [sp, #48]
	ldp x22, x21, [sp, #80]
	ldp x24, x23, [sp, #64]
	ldp x29, x30, [sp, #32]
	add sp, sp, #112
	ret
	mov x19, x1
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x1, x19
	mov x19, xzr
	b .LBB11_18
