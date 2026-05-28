chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 56
	mov r13, r8
	mov r15, rdx
	mov r14, rsi
	mov rbx, rdi
	lea rax, [rcx + 8]
	mov qword ptr [rsp + 16], rax
	xor ebp, ebp
	movabs r12, -4294967296
.LBB9_1:
	mov rax, r13
	jmp .LBB9_2
.LBB9_7:
	mov rax, r13
	lock cmpxchg	qword ptr [r14], rdi
	je .LBB9_8
.LBB9_2:
	mov r13, rax
	mov rsi, qword ptr [r14 + 408]
	mov rax, qword ptr [r14 + 416]
	mov rdx, rax
	and rdx, r13
	dec rsi
	cmp rdx, rsi
	seta sil
	sbb sil, 0
	je .LBB9_5
	movzx eax, sil
	cmp eax, 255
	jne .LBB9_12
	lea rdi, [r13 + 1]
	jmp .LBB9_6
.LBB9_5:
	or eax, r13d
	inc eax
	mov rdi, r13
	and rdi, r12
	or rdi, rax
.LBB9_6:
	mov esi, r13d
	mov rax, r13
	shr rax, 32
	cmp eax, r13d
	jne .LBB9_7
	mov rax, qword ptr [r14]
	cmp rax, r13
	jne .LBB9_2
	mov rax, qword ptr [r14 + 128]
	mov r8d, dword ptr [r14 + 416]
	add eax, r8d
	inc eax
	cmp rax, rsi
	je .LBB9_14
	mov edi, edi
	shl rax, 32
	or rax, rdi
	mov rdi, rax
	jmp .LBB9_7
.LBB9_14:
	test bpl, 1
	jne .LBB9_21
	cmp qword ptr [rcx], 0
	jne .LBB9_17
	lea rax, [r14 + 256]
	mov qword ptr [rcx], rax
	mov rax, qword ptr [rsp + 16]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	mov byte ptr [rcx + 40], 2
.LBB9_17:
	mov qword ptr [rsp + 24], r15
	mov qword ptr [rsp + 32], r15
	mov qword ptr [rsp + 40], 0
	mov rdi, rcx
	lea rsi, [rsp + 24]
	mov qword ptr [rsp + 8], rcx
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov rcx, qword ptr [rsp + 8]
	mov ebp, eax
	test al, al
	jne .LBB9_1
	cmp qword ptr [rcx], 0
	jne .LBB9_19
.LBB9_20:
	mov rcx, qword ptr [rsp + 8]
	mov qword ptr [rcx], 0
	jmp .LBB9_1
.LBB9_19:
	mov rdi, qword ptr [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_20
.LBB9_8:
	shl rdx, 4
	add rdx, qword ptr [r14 + 400]
	mov qword ptr [rbx + 8], rdx
	mov qword ptr [rbx + 16], rsi
	mov qword ptr [rbx], 0
	jmp .LBB9_13
.LBB9_12:
	xorps xmm0, xmm0
	movups xmmword ptr [rbx], xmm0
.LBB9_13:
	add rsp, 56
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_21:
	mov qword ptr [rbx], 1
	jmp .LBB9_13
	mov rcx, qword ptr [rsp + 8]
	mov qword ptr [rcx], 0
	mov rdi, rax
	call _Unwind_Resume@PLT
