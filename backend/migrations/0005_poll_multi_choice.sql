-- Drop single-answer constraint, allow multiple choices per user per poll
ALTER TABLE "EventPollAnswers" DROP CONSTRAINT "EventPollAnswers_pkey";
ALTER TABLE "EventPollAnswers" ADD PRIMARY KEY ("pollId", "userId", "choiceId");
