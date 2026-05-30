mpsc_recv_blocking:
	push rbx
	sub rsp, 48
	mov rbx, qword ptr [rdi]
	mov rcx, qword ptr [rbx + 256]
	mov rax, qword ptr [rbx + 544]
	and rax, rcx
	mov rsi, qword ptr [rbx + 528]
	shl rax, 4
	lea rdx, [rsi + rax]
	mov rax, qword ptr [rsi + rax + 8]
	cmp rax, rcx
	jne .LBB7_1
.LBB7_3:
	mov rsi, qword ptr [rbx + 536]
	mov rax, qword ptr [rbx + 544]
	mov rdi, rax
	and rdi, rcx
	dec rsi
	cmp rdi, rsi
	jne .LBB7_5
	or rcx, rax
.LBB7_5:
	inc rcx
	mov rdx, qword ptr [rdx]
	xchg qword ptr [rbx + 256], rcx
	mov rax, qword ptr [rbx + 384]
	test al, 1
	jne .LBB7_6
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB7_1:
	lea rsi, [rbx + 128]
	mov dword ptr [rsp + 40], 1000000000
	mov rdi, rsp
	lea r8, [rsp + 16]
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	mov rdx, qword ptr [rsp]
	test rdx, rdx
	je .LBB7_9
	mov rcx, qword ptr [rsp + 8]
	jmp .LBB7_3
.LBB7_6:
	add rbx, 384
	mov rdi, rbx
	mov rbx, rdx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	mov rdx, rbx
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB7_9:
	mov eax, 1
	add rsp, 48
	pop rbx
	ret
