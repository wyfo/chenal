mpmc_recv_blocking:
	push rbx
	sub rsp, 48
	mov rbx, qword ptr [rdi]
	mov rax, qword ptr [rbx + 256]
	mov rdx, qword ptr [rbx + 560]
	and rdx, rax
	mov rcx, qword ptr [rbx + 544]
	mov rsi, rdx
	shl rsi, 4
	mov rdi, qword ptr [rcx + rsi + 8]
	cmp rdi, rax
	jne .LBB11_9
	add rcx, rsi
	mov rcx, qword ptr [rcx]
	#MEMBARRIER
	mov rsi, qword ptr [rbx + 552]
	dec rsi
	cmp rdx, rsi
	jne .LBB11_3
	mov rdx, qword ptr [rbx + 560]
	or rdx, rax
	inc rdx
	lock cmpxchg	qword ptr [rbx + 256], rdx
	je .LBB11_5
.LBB11_9:
	mov dword ptr [rsp + 40], 1000000000
	lea rsi, [rbx + 128]
	mov rdi, rsp
	lea rcx, [rsp + 16]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	cmp byte ptr [rsp], 1
	jne .LBB11_6
	mov eax, 1
	add rsp, 48
	pop rbx
	ret
.LBB11_3:
	lea rdx, [rax + 1]
	lock cmpxchg	qword ptr [rbx + 256], rdx
	jne .LBB11_9
.LBB11_5:
	mov qword ptr [rsp + 8], rcx
.LBB11_6:
	mov rdx, qword ptr [rsp + 8]
	mov rax, qword ptr [rbx + 384]
	test al, 1
	jne .LBB11_7
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB11_7:
	add rbx, 384
	mov rdi, rbx
	mov rbx, rdx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	mov rdx, rbx
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
