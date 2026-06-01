mpsc_recv:
	push rbx
	sub rsp, 32
	mov rbx, qword ptr [rdi]
	mov r8, qword ptr [rbx + 256]
	mov rax, qword ptr [rbx + 544]
	and rax, r8
	mov rdx, qword ptr [rbx + 528]
	shl rax, 4
	lea rcx, [rdx + rax]
	mov rax, qword ptr [rdx + rax + 8]
	cmp rax, r8
	jne .LBB7_1
.LBB7_4:
	mov rdx, qword ptr [rbx + 536]
	mov rax, qword ptr [rbx + 544]
	mov rsi, rax
	and rsi, r8
	dec rdx
	cmp rsi, rdx
	jne .LBB7_6
	or r8, rax
.LBB7_6:
	inc r8
	mov rdx, qword ptr [rcx]
	xchg qword ptr [rbx + 256], r8
	mov rax, qword ptr [rbx + 384]
	test al, 1
	jne .LBB7_8
	xor eax, eax
	add rsp, 32
	pop rbx
	ret
.LBB7_1:
	lea rax, [rbx + 128]
	mov rdx, qword ptr [rsi]
	lea rdi, [rsp + 8]
	mov rsi, rax
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	mov eax, 2
	cmp byte ptr [rsp + 8], 0
	jne .LBB7_10
	mov rcx, qword ptr [rsp + 16]
	test rcx, rcx
	je .LBB7_9
	mov r8, qword ptr [rsp + 24]
	jmp .LBB7_4
.LBB7_8:
	add rbx, 384
	mov rdi, rbx
	mov rbx, rdx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	mov rdx, rbx
	xor eax, eax
	add rsp, 32
	pop rbx
	ret
.LBB7_9:
	mov eax, 1
.LBB7_10:
	add rsp, 32
	pop rbx
	ret
