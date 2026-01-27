;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="enum" href="tower_governor/errors/enum.GovernorError.html" title="enum tower_governor::errors::GovernorError">GovernorError</a>',
					1,
					['tower_governor::errors::GovernorError'],
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/key_extractor/struct.GlobalKeyExtractor.html" title="struct tower_governor::key_extractor::GlobalKeyExtractor">GlobalKeyExtractor</a>',
					1,
					['tower_governor::key_extractor::GlobalKeyExtractor'],
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/key_extractor/struct.PeerIpKeyExtractor.html" title="struct tower_governor::key_extractor::PeerIpKeyExtractor">PeerIpKeyExtractor</a>',
					1,
					['tower_governor::key_extractor::PeerIpKeyExtractor'],
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/key_extractor/struct.SmartIpKeyExtractor.html" title="struct tower_governor::key_extractor::SmartIpKeyExtractor">SmartIpKeyExtractor</a>',
					1,
					['tower_governor::key_extractor::SmartIpKeyExtractor'],
				],
				[
					'impl&lt;F&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/struct.ResponseFuture.html" title="struct tower_governor::ResponseFuture">ResponseFuture</a>&lt;F&gt;<div class="where">where\n    F: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,</div>',
					1,
					['tower_governor::ResponseFuture'],
				],
				[
					'impl&lt;K, M&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfig.html" title="struct tower_governor::governor::GovernorConfig">GovernorConfig</a>&lt;K, M&gt;<div class="where">where\n    K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    &lt;K as <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>&gt;::<a class="associatedtype" href="tower_governor/key_extractor/trait.KeyExtractor.html#associatedtype.Key" title="type tower_governor::key_extractor::KeyExtractor::Key">Key</a>: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a>,</div>',
					1,
					['tower_governor::governor::GovernorConfig'],
				],
				[
					'impl&lt;K, M&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfigBuilder.html" title="struct tower_governor::governor::GovernorConfigBuilder">GovernorConfigBuilder</a>&lt;K, M&gt;<div class="where">where\n    K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,</div>',
					1,
					['tower_governor::governor::GovernorConfigBuilder'],
				],
				[
					'impl&lt;K, M&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/struct.GovernorLayer.html" title="struct tower_governor::GovernorLayer">GovernorLayer</a>&lt;K, M&gt;<div class="where">where\n    K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    &lt;K as <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>&gt;::<a class="associatedtype" href="tower_governor/key_extractor/trait.KeyExtractor.html#associatedtype.Key" title="type tower_governor::key_extractor::KeyExtractor::Key">Key</a>: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a>,</div>',
					1,
					['tower_governor::GovernorLayer'],
				],
				[
					'impl&lt;K, M, S&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> for <a class="struct" href="tower_governor/governor/struct.Governor.html" title="struct tower_governor::governor::Governor">Governor</a>&lt;K, M, S&gt;<div class="where">where\n    K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    S: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a>,\n    &lt;K as <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>&gt;::<a class="associatedtype" href="tower_governor/key_extractor/trait.KeyExtractor.html#associatedtype.Key" title="type tower_governor::key_extractor::KeyExtractor::Key">Key</a>: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html" title="trait core::marker::Send">Send</a> + <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html" title="trait core::marker::Sync">Sync</a>,</div>',
					1,
					['tower_governor::governor::Governor'],
				],
			],
		],
	])
	if (window.register_implementors) {
		window.register_implementors(implementors)
	} else {
		window.pending_implementors = implementors
	}
})()
//{"start":57,"fragment_lengths":[7317]}
