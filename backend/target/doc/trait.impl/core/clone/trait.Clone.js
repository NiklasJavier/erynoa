;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="enum" href="tower_governor/errors/enum.GovernorError.html" title="enum tower_governor::errors::GovernorError">GovernorError</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/key_extractor/struct.GlobalKeyExtractor.html" title="struct tower_governor::key_extractor::GlobalKeyExtractor">GlobalKeyExtractor</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/key_extractor/struct.PeerIpKeyExtractor.html" title="struct tower_governor::key_extractor::PeerIpKeyExtractor">PeerIpKeyExtractor</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/key_extractor/struct.SmartIpKeyExtractor.html" title="struct tower_governor::key_extractor::SmartIpKeyExtractor">SmartIpKeyExtractor</a>',
				],
				[
					'impl&lt;K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> + <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>, M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> + RateLimitingMiddleware&lt;QuantaInstant&gt;&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfig.html" title="struct tower_governor::governor::GovernorConfig">GovernorConfig</a>&lt;K, M&gt;<div class="where">where\n    K::<a class="associatedtype" href="tower_governor/key_extractor/trait.KeyExtractor.html#associatedtype.Key" title="type tower_governor::key_extractor::KeyExtractor::Key">Key</a>: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a>,</div>',
				],
				[
					'impl&lt;K: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> + <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>, M: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> + RateLimitingMiddleware&lt;QuantaInstant&gt;&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfigBuilder.html" title="struct tower_governor::governor::GovernorConfigBuilder">GovernorConfigBuilder</a>&lt;K, M&gt;',
				],
				[
					'impl&lt;K: <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>, M: RateLimitingMiddleware&lt;QuantaInstant&gt;&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/struct.GovernorLayer.html" title="struct tower_governor::GovernorLayer">GovernorLayer</a>&lt;K, M&gt;',
				],
				[
					'impl&lt;K: <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>, M: RateLimitingMiddleware&lt;QuantaInstant&gt;, S: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a>&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html" title="trait core::clone::Clone">Clone</a> for <a class="struct" href="tower_governor/governor/struct.Governor.html" title="struct tower_governor::governor::Governor">Governor</a>&lt;K, M, S&gt;',
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
//{"start":57,"fragment_lengths":[4443]}
